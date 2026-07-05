---
scope: build/qa
audience: (see scope README)
source: private memory
  `discriminator-negative-arm-must-be-expressible-and-reaching`
---

# A discriminator's negative arm can be vacuous if nothing can express it

The per-dimension rule (K2c-series-2: author a discriminating case per
dimension) has a **prerequisite** the case itself can't self-check: the
discriminating input — especially the **negative/insufficient** arm — must be
**expressible by the upstream deliverable AND reach the target gate**. A case
can be vacuous (green-vs-green) not because you authored it wrong, but because
the upstream mechanism that should *produce* the negative input either can't
express it or produces something that fails **earlier**, for a different reason,
never reaching the gate the case targets.

**Why:** fs-read-file-lines-flip AC4 (CV D4/D5). The discriminator is "CLI
grants **exactly** the declared authority, not full." The negative arm needs a
`main` declaring **insufficient** authority whose cap **reaches the driver** and
is denied at `authorizes`. Grounded facts made this a knife-edge: read requires
`AUTH_PARTIAL(1)`, the **only** sub-required level is `AUTH_NONE(0)`, and there
is **no FS write op** to use as an alternative "present-but-insufficient" lever.
So if D2's manifest treated "declares `ANone`" as "**provide no cap**," the
insufficient `main` would fail at bind/`NotAnIOTree` (wrong reason) or
`MissingCapability`-at-elaboration — **never at the driver's authority gate** —
and AC4 collapses to vacuous. The discriminator's realizability was **contracted
by D2's design**, an upstream deliverable I did not author. I surfaced it as
**SEAM-A, load-bearing, at plan time** (before D2 was finalized); spec-author
then designed `read_bytes` authority-**polymorphic** (design α) so an `ANone`
cap is minted+bound and reaches the driver, and Architect confirmed **α is
*forced* by the locked AC4** ("refused **at the driver**"). Front-loaded to the
cheapest point instead of surfacing as a dead AC at build.

**How to apply:** (1) For each discriminating dimension, don't stop at "I
authored a case" — trace the **negative arm's input** from its **producer**:
*can the upstream deliverable express it, and does it reach the gate this case
targets, or fail earlier for a different reason?* (2) The danger sign: the only
way to be "insufficient/negative" is a **single boundary value** the upstream
might **elide** (collapse to "provide nothing" / reject earlier). With no
**alternative lever** (a second op at a higher threshold, another failure axis),
that one value is the whole net — pin its expressibility as a **load-bearing
dependency on the upstream**, at **plan time**, routed to the upstream author,
not discovered at build. (3) If the upstream genuinely can't express it, that's
a real fork → escalate, don't quietly ship a vacuous AC. Extends conformance
hand feeds the deliverable (there the *positive* deliverable is hand-fed; here
the *negative* input can't be produced) and the K2c-series-2 per-dimension rule
(from "a case per dimension" to "each dimension's discriminating value is
expressible-and-reaching before the build starts"). Sibling caution: your own
companion plan can also go **stale** when the upstream design **narrows** under
it (frame `Cap FS` → landed `Cap a`) — the fidelity vote's content-reconcile
against the **landed body** is what catches your own artifact's staleness
(disclaimed framing still binds your own companion artifact).
