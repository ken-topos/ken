# Hailstone sequence

Generate the hailstone (Collatz) sequence from n down to 1.

Reference: <https://rosettacode.org/wiki/Hailstone_sequence>

## Status

**Working, end to end.** `ken run` prints `PASS` and exits 0. Was
blocked pending `wp/RTP1-interp-sharing` (`isEven`/`half`/`isOne`
independently walking `n` at every step hit the eager-IH-computation
cost) — resolved (`e88ffa8`), confirmed fast.

## Implementation notes

- The Collatz conjecture (every `n` eventually reaches 1) is an open
  mathematical problem — no termination proof (structural or otherwise)
  can exist for an unbounded `hailstone`. `hailstoneFueled` uses an
  explicit `Nat` fuel bound, the same idiom as this corpus's own
  `gcd.ken`/`merge-sort.ken` — a real, total-by-construction function
  that computes the true sequence for any input whose real trajectory
  fits within the fuel.
- `half`/`isEven` use a double-nested-match-then-recurse shape (same
  idiom as this corpus's own `natSub`) to peel 2 `Suc`s per step without
  needing a `-`/`/` infix operator (neither exists — `GAP-subtraction`).
- `3n+1` is computed via `natAdd` only (`n+n+n+1`), avoiding the
  (separately known) missing `mul_int`/multiplication-on-`Nat` gap.
- Discriminating oracle: the real trajectory for `n=6`
  (`6,3,10,5,16,8,4,2,1`), checked via structural list equality.

## Oracle

`main` prints `PASS`.
