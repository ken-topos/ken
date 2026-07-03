# KNOWN-GAP: GAP-natToDecimal-range — printing bare `n` for 1..20 is
# infeasible

## What's missing

FizzBuzz's `Plain` case (`n` not divisible by 3 or 5) needs to print the
bare decimal number itself (`"1"`, `"2"`, `"4"`, `"7"`, ...). The only
existing decimal-conversion approach (`natToDecimal`, the VAL2
`natToDecimal`/`gcd` finding) has a confirmed **exponential** cost in
`ken-interp` (a bound value referenced twice in one function body — the
same root cause already being fixed in `wp/RTP1-interp-sharing`). Growth
data from that finding (`gcd`/`merge-sort` READMEs): single digits are
fine, but cost climbs roughly 3-8x per unit increase — printing every
value 1..20 (including two-digit numbers) is well past the point where
this is a "slow but shippable" cost, unlike `gcd`'s single-digit result.

## What's NOT the gap (closed since this file was first written)

- `print_line`/`ken run` Console execution: landed.
- `Bool` pattern-matching: works directly (`match b { True => .. ; False =>
  .. }`, confirmed via `packages/collections/collections.ken`'s
  `compareChar`).
- String literals: landed (VAL1-surface).
- IO-sequencing / looping over a range via recursion: confirmed
  SCT-acceptable (VAL2's `printLoop` probe — a `Vis`-under-closure
  recursive print combinator elaborates and evaluates cheaply even at
  depth 20). This was NOT the blocker once probed.

The classification logic (`Mod3`/`Mod5`/`classify` in `fizzbuzz.ken`) is
unaffected by any of this and elaborates + evaluates correctly on its own
(the `GAP-nested-patterns` accumulator-type workaround it uses is a
separate, already-landed workaround, not this gap).

## Fix needed

Resolves automatically once `wp/RTP1-interp-sharing` (Steward-scoped,
Runtime team, in flight) lands call-by-need substitution sharing in
`ken-interp` — the same fix already scoped for the `natToDecimal`/
`merge-sort` family. Not a Language-lane fix, not patched here.

## Intended program (once resolved)

Combine the already-working `classify`, a decimal `natToDecimal`, and the
already-probed `printLoop`-style recursive IO combinator to iterate 1..20,
printing each classification result (`"Fizz"`/`"Buzz"`/`"FizzBuzz"`/the
decimal `n`).
