---
scope: fleet
audience: (see scope README)
source: private memory `live-review-candidate-goes-stale-reanchor-sha`
---

# A live review candidate can go stale; re-anchor the SHA

Two coupled review-discipline lessons from the `map-convoy-idiom` (Map capstone
Unit 1) Spec gate.

**1. A live review candidate can go stale between read and vote.** I began
reviewing `54` at `d372429` and independently reached two findings (an
"undetermined proof-bug-vs-gap" binary + a `check_match_dependent` attribution
Architect had *just refuted*). While I read, a **grounding storm** resolved in
parallel and the author folded a fix at `181f6f6` — so both my findings were
**already corrected** in a candidate I hadn't fetched. Casting off the
first-read SHA would have posted a **stale finding against a superseded
candidate**. This is check main via git object store not find / git-wins
extended from *main-checks* to *live review candidates*: on every new mention
during a review, **re-fetch + re-diff to the live tip before casting**, and diff
old-tip..new-tip to see exactly what moved. A vote is only valid against the SHA
you actually re-derived.

**2. Honesty-boundary-held ≠ WHY-prose-fresh — don't collapse them into a
block.** When a WP is authored to an explicit instruction ("mark X PENDING,
Architect grounding it") and that grounding **completes before merge**, the
doc's *explanatory prose* (the WHY) goes stale even while its **honesty-boundary
(the AC)** stays correct — the doc still doesn't over-claim buildability. The
catch-up is a **doc-freshness reconcile, NOT an AC failure**. As reviewer, hold
two verdicts distinct: "honesty boundary held → APPROVE-worthy" vs "WHY-prose
stale / mis-attributed → fold-in." A stale mechanism attribution (here: the
residual pinned to `check_match_dependent` when the real gap was `conv_struct`'s
missing `(Eq,Eq)` congruence arm) is worth a fold because it would misdirect a
future reader to the wrong file — but it is not a soundness/honesty block.
Over-blocking a boundary-honest doc for a freshness lag is as wrong as missing
an over-claim.

**Why it matters:** a wrong conformance/spec verdict licenses wrong work
fleet-wide; but a *needless block* on a boundary-honest doc stalls a clean
merge. Both errors are real. The discipline that avoids both: re-anchor to the
live SHA, then classify each defect as boundary-violating (block) vs prose-stale
(fold). In this WP the author's own `d372429→181f6f6` fold caught exactly the
two sites I'd independently flagged, so my finding **converged with the fold**
and I approved `181f6f6` cleanly instead of blocking a stale read.

Sibling of reconcile proof rides elaboration merge not build phase (ground-truth
moving during authoring) and the "content-reconcile is necessary but not
sufficient — re-derive from first principles" rule.
