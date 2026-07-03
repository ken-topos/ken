# KNOWN-GAP: printing 5! = 120 is infeasible (natToDecimal family)

## What's missing

`main` computes `factorial 5 = 120` correctly (`natToInt (factorial
five)`, verified in-process) but does not print it via IO. Decimal
`Nat`->`String` conversion (`natToDecimal`) is confirmed infeasible for a
value this size — see `examples/rosetta/gcd/KNOWN-GAP.md` for the full
finding (the `natToDecimal`/`merge-sort` exponential family, further
generalized to a whole-program-size sensitivity, routed to
language-leader/Steward/Architect, fix in flight on
`wp/RTP1-interp-sharing`). `120` is a two-digit number well past the
single-digit range where that machinery was even marginally tractable
pre-fix.

## Fix needed

Same fix already scoped and in flight: `wp/RTP1-interp-sharing`. Not a
Language-lane fix, not patched here.

## Intended program (once resolved)

`view main : IO Unit = print_line (natToDecimal (natToInt (factorial
five)))` (or a `Nat`-native decimal conversion, once one exists) — see
`gcd.ken`'s commented-out reference implementation of the decimal-printing
machinery for the shape.
