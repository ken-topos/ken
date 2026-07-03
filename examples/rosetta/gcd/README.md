# Greatest common divisor

Compute gcd(a, b) for two non-negative integers.

Reference: <https://rosettacode.org/wiki/Greatest_common_divisor>

## Status

**GCD algorithm is correct; blocked pending `wp/RTP1-interp-sharing`, see
`KNOWN-GAP.md`.** Not a Language-lane gap — a `ken-interp` evaluation-cost
characteristic (Architect-ruled root cause: `elim_reduce`'s eager,
unconditional IH computation) currently makes even this file's plain
`natGcd` computation (no printing at all) fail to complete in reasonable
time once combined with the file's own ~20 declarations. Fix is in
flight; this file's algorithm needs no changes once it lands.

## Implementation notes

- Subtraction-based GCD with explicit fuel (`natGcdFueled`, fuel = `a+b`)
  for SCT termination.
- `GAP-nested-patterns`: `Suc _` in `natCmp` (a nested constructor
  pattern) triggers a reachability error — worked around via a
  non-recursive `natCmpZero` helper with only flat patterns.
- A decimal-printing attempt (`natToDecimal`) was built, found infeasible,
  and kept as commented-out reference code — see `KNOWN-GAP.md` for the
  full history (it surfaced the broader finding above, not just a
  printing-specific one).

## Oracle

None (`KNOWN-GAP.md`) — blocked on verification, not on correctness.
