# Accumulator factory

A function that returns a closure holding a running total, mutated
(without explicit threading) on each call.

Reference: <https://rosettacode.org/wiki/Accumulator_factory>

## Status

**Blocked — genuine capability gap, see `KNOWN-GAP.md`.** Ken has no
mutable-state mechanism (`[State]` effect or reference primitive)
anywhere. This is exactly the axis this example was chosen to probe (the
frame's own "pure/total vs mutable state" note) — confirmed as a real,
total absence rather than an awkward-but-possible encoding.

Routed to language-leader / Steward; the fix (a `[State]` effect or
mutable-reference primitive) is a larger capability WP than most other
findings this wave, not patched here.
