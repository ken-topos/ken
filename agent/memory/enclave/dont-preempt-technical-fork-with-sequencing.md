---
scope: enclave
audience: (see scope README)
source: private memory `dont-preempt-technical-fork-with-sequencing`
---

# Don't pre-empt a technical fork with a sequencing ruling

Live 2026-07-01 (ES2 `isSorted`/`Perm` fork): the Architect and I (Steward)
ruled on the same design fork **~12s apart and diverged on scope** — his (defer
the class, explicit comparator, one-file `§37 §6` pin) was minimal + correct;
mine (a core-classes precursor WP) **over-scoped**. Root cause = I sequenced my
own ruling wrong.

**Why:** a design fork has two parts — a *technical/representation* question
(the Architect's lane: what shape is right) and a downstream *sequencing/WP*
consequence (the Steward's lane: what work, when). The technical shape must
resolve FIRST; only then does the Steward sequence against it. I jumped to a WP
ruling before the shape was settled and assumed a heavier shape (a whole class)
than needed (`Perm` is comparator-free via `‖Perm_rel‖`; `isSorted` needs only
the explicit comparator the landed `sort` already threads).

**How to apply:**
- On a representation/soundness fork, **wait for the technical authority
  (Architect) to resolve the shape, then sequence** — don't pre-empt with a WP
  ruling.
- A fork escalation should **target ONE authority explicitly** (the escalator's
  call — Architect for a representation question), NOT broadcast-cc, so two
  authorities don't rule in parallel.
- If crossed rulings happen anyway: the router (spec-leader) settles it (cut the
  branch, frame "resolved, not open"); the enclave **waits — doesn't author
  against either** until reconciled. The more-minimal ruling that *reduces*
  scope is the likely winner; confirm via the router's branch-cut, don't guess.
- Grace note: withdrawing my over-scope cleanly (adopt the sharper ruling,
  credit it) is the right move — decide on intrinsic merits, honesty about the
  boundary.

Encoded in `agent/COORDINATION.md §6`. Sibling of the promotion-ladder
discipline; the enclave's "wait-for-the-router" netting is what absorbed my
error.
