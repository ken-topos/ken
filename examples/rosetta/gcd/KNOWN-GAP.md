# KNOWN-GAP: blocked pending `wp/RTP1-interp-sharing` — `natGcd`'s own
# fuel-driven recursion pays the eager-IH tax at scale

## What's missing

This file's actual purpose (compute `gcd(12, 8) = 4`) is unaffected by any
Language-lane gap — the algorithm (`natGcdFueled`, subtraction-based with
explicit fuel) is correct and elaborates fine. The problem is purely a
`ken-interp` evaluation-performance characteristic, root-caused by
Runtime/Architect on `wp/RTP1-interp-sharing`: `elim_reduce` eagerly,
unconditionally computes an induction hypothesis on every match/elim
reduction step regardless of whether the method body actually consumes
it (discarded via `apply`'s `_ => Neutral` when unused) — an O(cost) tax
paid per step, independent of the specific function being reduced.

## Confirmed empirically (corrected understanding, per Runtime's D1 gate)

Earlier revisions of this file attributed the slowdown to "prepending
unrelated declarations" / "total declaration count." **That framing was
falsified by Runtime's isolating tests** (posted 2026-07-03,
`wp/RTP1-interp-sharing` D1): prepending 0/5/10/20/30 wholly-**unused**
declarations to an unrelated `natToDecimal` call cost **nothing**
(`elim_reduce` call count identical at every pad size) — raw declaration
count in the environment is not a per-step cost driver on its own.

The real driver: `natGcd`'s own fuel is `natAdd a b = 20` — a
much larger unary `Suc`-chain than the small literals this repro's other
probes tested. `natGcdFueled`'s recursion over that 20-deep fuel pays the
same eager-IH tax on **its own** reduction, independent of anything
prepended around it. Confirmed directly: `natToInt (natGcd twelve eight)`
alone (no `natToDecimal`, no prepended anything) did not complete within
60-120s in an isolated harness. "3 unrelated decls prepended" in the
original repro wasn't testing environment size at all — it was testing a
program whose `main` argument (`natGcd twelve eight`) executes real,
fuel-scaled recursive code.

## Impact

This file cannot currently be verified end-to-end (elaborated AND
evaluated to confirm `natGcd 12 8 = 4`) in reasonable time via `ken run`
or any full-evaluation harness — not a bug in this file's algorithm, and
not specific to printing.

## Fix needed

`wp/RTP1-interp-sharing` (Runtime team, root cause confirmed single/
unified — see D1 close-out; fix: make the IH computation lazy/conditional
on actual consumption). Not a Language-lane fix, not patched here. The
exact repro (this file's `natAdd`/`natSub`/`natCmp`/`natGcdFueled`/
`natGcd`, fuel=20) is a pinned regression item for Runtime's AC4/AC5 gate.

## Intended program (once resolved)

`main` as currently written (`natToInt (natGcd twelve eight)`, evaluating
to `4`) should become fast once `wp/RTP1-interp-sharing` lands; the
commented-out decimal-printing machinery at the bottom of `gcd.ken`
documents the further intended `IO Unit`-printing `main`, to be re-added
once both this and the printing-specific finding resolve.
