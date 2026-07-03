# Ackermann function

Compute `A(m, n)` for small `m, n`.

Reference: <https://rosettacode.org/wiki/Ackermann_function>

## Status

**Blocked — pre-existing, self-documented `GAP-ackermann-sct`** (Ken's SCT
cannot infer lexicographic termination). See `KNOWN-GAP.md`.

**VAL2 fix:** this dir previously carried a stale `expected` file
(`29`) alongside a placeholder `main = Zero` — the oracle could never
match (the real `ack` definition is commented out, blocked by the gap
above). Per the runner's own contract (a dir must declare its oracle via
exactly one of `expected`/`KNOWN-GAP.md`, never a permanently-failing
`expected`), replaced it with `KNOWN-GAP.md`. `GAP-ackermann-sct` itself is
untouched — out of scope for this light-gated mini-WP, routed to
language-leader / Steward as a capability gap.

## Oracle

None (`KNOWN-GAP.md` — recorded finding, not CI-blocking). The intended
oracle, once resolved, is `A(3, 2) = 29`.
