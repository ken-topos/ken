---
scope: enclave
audience: (see scope README)
source: private memory `reconcile-proof-rides-elaboration-merge-not-build-phase`
---

# A reconcile fix can land in the elaboration merge, not the build phase

When a WP runs in **two phases** — an enclave **elaboration merge** (spec
skeleton + `/spec` reconciles + conformance-seed design) followed by a later
**build merge** (the actual proof/impl terms) — do **not** assume that *all*
code changes fall on the build side. A small **reconcile** code change (a
one-line proof, a narrative-fixing addition) can ride the **elaboration** merge
itself. A conformance seed authored for the elaboration phase that pins such a
thing as **red-until-built pending the build** is then **stale the instant it
lands** — the pinned artifact is realized-on-`main` immediately, and the seed
needs a same-hour correction.

**Why:** `map-verified-laws` (Map capstone). Its split was "enclave elaborates
the 5-inductive-law proof skeleton + reconciles → merge → Foundation builds the
5 proof terms." I framed **`orderedEmpty` (`Ordered empty = tt`)** as
red-until-built pending Foundation throughout my first seed, on the assumption
that *every* `packages/collections/map.ken` change is Foundation's build-phase
work. Wrong: `orderedEmpty` was a **Discrepancy-1 reconcile** (make the "two
shipped Branch-A proofs" narrative true), and spec-author landed it as a real
`view orderedEmpty … = tt` in the **elaboration commit** (`df81689`), verified
elaborating — so it shipped with *that* merge, not the later build. Only the
**five inductive laws** genuinely stayed red-until-built. I caught it myself
(reading `df81689` + the author's confirm) *and* the spec-leader independently
flagged the identical drift before opening the Decision. Fix: flip
`orderedEmpty` → **realized-at-this-merge** (sibling of `lookup empty`) wherever
it appeared (Status para, the delta-net `given`/flip), keep the five inductive
laws red.

**How to apply:** on a two-author / two-phase WP, before tagging any artifact
`red-until-built pending phase-2`, **grep which commit each code change actually
lands in** (`git show <elaboration-SHA> -- <impl-path>`), not which phase the
frame *assigned* it to. Reconciles that "make the narrative true" are the
classic case that rides the earlier merge. This is the phase-boundary corollary
of the red-until-built discipline: `red-until-built` is correct **only** for
artifacts that land in a *later* merge — an artifact co-landing in *this* merge
must be pinned **realized**, or the seed over-defers and reads as a false `main`
state the hour it merges. Rebasing my seed branch onto the elaboration tip
(`df81689`) made the "realized" pin **literally true on-branch** and let the
crate-byte-identity check (`git diff <tip> HEAD` outside my file = empty) prove
zero drift. Sibling of check main via git object store not find (verify against
the real ref/commit, not an assumption) and the buildability classify every
capability axis empirical-over-armchair rule.
