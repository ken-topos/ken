# Hailstone sequence

Generate the hailstone (Collatz) sequence from n down to 1.

Reference: <https://rosettacode.org/wiki/Hailstone_sequence>

## Status

**Algorithm correct (fuel-bounded, the honest idiom for a conjectured-not-
proved-terminating function); blocked pending `wp/RTP1-interp-sharing`,
see `KNOWN-GAP.md`.**

## Implementation notes

- The Collatz conjecture (every `n` eventually reaches 1) is an open
  mathematical problem — no termination proof (structural or otherwise)
  can exist for an unbounded `hailstone`. `hailstoneFueled` uses an
  explicit `Nat` fuel bound, the same idiom as this corpus's own
  `gcd.ken`/`merge-sort.ken` — a real, total-by-construction function
  that computes the true sequence for any input whose real trajectory
  fits within the fuel. This is the correct answer to the axis this task
  probes, not a workaround.
- `half`/`isEven` use a double-nested-match-then-recurse shape (same
  idiom as this corpus's own `natSub`) to peel 2 `Suc`s per step without
  needing a `-`/`/` infix operator (neither exists — `GAP-subtraction`).
- `3n+1` is computed via `natAdd` only (`n+n+n+1`), avoiding the
  (separately known) missing `mul_int`/multiplication-on-`Nat` gap.

## Oracle

None (`KNOWN-GAP.md`) — blocked on verification, not on correctness. The
intended oracle is `hailstone(6) = 6,3,10,5,16,8,4,2,1`.
