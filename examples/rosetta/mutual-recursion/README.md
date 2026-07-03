# Mutual recursion

Classic even/odd via two mutually-recursive functions.

Reference: <https://rosettacode.org/wiki/Mutual_recursion>

## Status

**Blocked — genuine capability gap, see `KNOWN-GAP.md`.** Ken's surface has
no construct for declaring a group of mutually-recursive `view`s; confirmed
empirically that neither declaration order elaborates (each fails with
`UnresolvedCon` on the not-yet-declared name). This is exactly the axis this
example was chosen to probe (`docs/program/wp/VAL2-rosetta-pangram.md` — "SCT
mutual recursion (`declare_recursive_group` at the surface); tiny/high-signal")
— the gap it surfaced is real, not a workaround-able one.

Routed as a finding to language-leader / Steward; the fix (a `mutual`
surface construct wired through `declare_recursive_group` with a
group-spanning termination measure) is its own properly-gated capability WP,
not patched here.
