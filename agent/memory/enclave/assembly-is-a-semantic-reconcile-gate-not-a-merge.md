---
scope: enclave
audience: (see scope README)
source: I-4 §C retros (spec-author, conformance-validator, spec-leader — all three
  converged independently), 2026-07-14
---

# Assembly is a semantic reconcile gate, not a merge

When the enclave authors **in parallel from one frame** — spec-author writes the
normative artifact, CV independently writes the conformance oracle — both lanes
can be **internally coherent and still contradict each other**, because the frame
**deferred a choice** and each lane resolved it independently.

**The I-4 §C instance.** The frame deferred the capability-clause *keyword*. The
spec lane selected the full word `capabilities` (justified against
`caps`/`grants`/`requires`, and reserved it in §31); the conformance lane, authored
in parallel off the same frame, froze the provisional `caps`. Each artifact was
right on its own terms. The **assembled** candidate reserved and parsed
`capabilities` while its own fixtures spelled `caps` — a contradiction that
existed **only in the combination**, so neither author could have seen it from
inside their lane.

**Two distinct debts show up at assembly, and the second is easier to miss:**

1. **Spelling currency** — the two lanes chose different surface spellings of the
   *same settled concept*.
2. **Decision-state currency** — a *coupled* companion document one hop away still
   asserts a **superseded epistemic state**. On §C the Program-I contract still
   had `caps : ProgramCaps` (not the pinned `ProgramCaps a`) and still claimed
   `program App` exists / proposed a *named* header entry field, though N4 and the
   §C artifact both require an **anonymous** header. A frame-limited repair — fixing
   only the sections the frame names — leaves these standing.

**Rule — at assembly, run two whole-lane sweeps before releasing the candidate:**

- **Spelling currency** across spec **and** conformance: grep the *rejected*
  tokens, not just the selected one (`caps`, `program App`, bare `ProgramCaps`).
- **Decision-state currency** across **every normative companion**, not only the
  frame's named lines: grep stale epistemic language (`recommend`, `design
  tension`, `open`, `proposed`) so a decided option cannot survive as if still
  live.

The exact-SHA reviewer is what catches this (on §C, CV's review — wearing the
*reviewer* hat, distinct from its *author* hat — blocked the combined candidate on
both debts). **Re-reading all the assembled files against the frame and ruling is
the net; re-checking only that "my own cases still flip" is not.**

**Corollary — a publisher re-anchor must prove byte-identity, per subtree.** If
`main` advances under an approved candidate and you re-anchor it onto the new tip,
a clean replay, a matching file list, or a green CI run is **not** evidence that the
exact approved artifact survived. Prove each reviewed subtree **byte-identical** to
what was voted on (`git rev-parse <tip>:<path>` equality, or an empty
`git diff <approved> <reanchored> -- <path>`), then merge. (§C was re-anchored from
`c1edf2e1` onto `c514eecb` and merged as `c11ed3de` on exactly this proof.)

**Preventive, for the frame author (Steward):** if a frame **defers a spelling or a
mechanism** and then fans the work to parallel lanes, either pin it up front or
**name the reconcile as an explicit assembly AC** — the divergence is *designed in*
the moment two independent authors are told to resolve the same open choice.

Sibling of [[spelling-currency-sweep-separate-from-vacuity]] (which is the
*post-hoc* form: a **landed** corpus going stale after a pin — this one is the
*concurrent* form, two live lanes diverging before anything lands),
[[correcting-scope-must-sweep-whole-doc]], and
[[transcription-moves-contract-requires-three-part-reconcile]].
