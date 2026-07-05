---
scope: fleet
audience: (see scope README)
source: private memory `handoff-scope-count-must-match-full-thread`
---

# Verify a handoff's scope count against the full thread

A build handoff's enumerated scope count (e.g. "the seed minus the two
explicitly-deferred items") can go stale mid-thread even when the delivered code
is fully correct — a terminal merge_ready message can carry an earlier count
forward after a LATER ruling added another deferred item, if the implementer
never re-synced the summary prose to the final scope.

**Why:** multi-step scope forks (a soundness tension discovered, ruled, then a
SECOND coupled tension discovered later touching the same axis) each correctly
shrink what ships, but only the LAST ruling reflects true final scope — earlier
"N items deferred" phrasing is easy to leave unedited in a closing summary once
N grows.

**How to apply:** verify a claimed deferral/scope count two ways, not one: (1)
by absence in the actual diff (grep for the law instance / stubbed proof /
removed feature and confirm it's genuinely not there — this nets the substance),
AND (2) by re-deriving the count from the full thread's rulings, not the
handoff's own summary line. A message can be internally consistent and still
wrong if drafted before the last scope change. Live catch: WP
decimal-char-demote-build's merge_ready said "two explicitly-deferred items" but
the thread had ruled THREE (AC-D3 Decimal law instance, `Ord Char` antisym,
pin-2 extraction) — substance was correct, only the count was stale; implementer
confirmed the correction immediately, non-blocking.

An inaccurate count in a hard-AC enumeration risks a downstream gate reviewer
misreading "expect 2 absences" and getting confused hitting a 3rd. Sibling of
trust level prose vs locked adr crosscheck (prose can under/over-claim relative
to what's actually grounded) and conformance hand feeds the deliverable (verify
the real producer, not the summary) — this applies the same
discriminating-verification instinct to the SCOPE FRAMING itself, not just the
code.
