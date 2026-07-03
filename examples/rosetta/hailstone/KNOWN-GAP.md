# KNOWN-GAP: blocked pending `wp/RTP1-interp-sharing`

## What's missing

The algorithm (`hailstoneFueled`, fuel-bounded per the totality/fuel idiom
this task is meant to probe — see the file header) is correct and
elaborates fine. It cannot currently be evaluated in reasonable time,
same root cause as `examples/rosetta/gcd/KNOWN-GAP.md`: `ken-interp`'s
`elim_reduce` eagerly, unconditionally computes an induction hypothesis
on every match/elim reduction step regardless of whether it's consumed.

`hailstoneFueled` calls `isOne`/`isEven`/`half` at every step, each
independently walking `n` from scratch via its own recursive structural
match — the same "multiple nested recursive helpers per outer step"
shape that made `natGcd`'s fuel-driven recursion pathological (see the
`gcd` finding). Confirmed empirically: evaluating `hailstoneFueled` on
this file's small oracle input (`n=6`, sequence length 9, no value ever
exceeding 16) did not complete within 60 seconds.

## Impact

Cannot be verified end-to-end today, purely due to the tracked
`ken-interp` characteristic — not a bug in the algorithm, and not
specific to the fuel/totality idiom this task probes (that idiom is
correct and is exactly what a total language requires here).

## Fix needed

Same fix already scoped and in flight: `wp/RTP1-interp-sharing`. Not a
Language-lane fix, not patched here.

## Intended program (once resolved)

`hailstone.ken` as currently written should become fast once
`wp/RTP1-interp-sharing` lands — no changes needed to the algorithm
itself.
