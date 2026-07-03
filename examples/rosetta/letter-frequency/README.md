# Letter frequency

Count how many times each letter occurs in a string.

Reference: <https://rosettacode.org/wiki/Letter_frequency>

## Status

**Blocked — genuine capability gap, see `KNOWN-GAP.md`.** `Map` is
declared as a type but has zero operations anywhere — confirmed by
inspection, not just "awkward without it." This is exactly the axis this
example was chosen to probe (the frame's own "Map gap" note) — the gap it
surfaced is as severe as expected.

Routed to language-leader / Steward; the fix (wiring real `Map`
primitive operations in `ken-elaborator`/`ken-interp`) is its own
properly-gated capability WP, not patched here.
