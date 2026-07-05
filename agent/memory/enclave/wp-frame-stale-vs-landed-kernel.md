---
scope: enclave
audience: (see scope README)
source: private memory `wp-frame-stale-vs-landed-kernel`
---

# A WP frame's description of the kernel can be stale vs the landed kernel

A **Steward/WP frame** is authored at a point in time, and the **kernel moves
under it** — especially via Architect follow-up soundness fixes between the
frame's writing and the WP's pickup. So the frame's description of *"what is
currently deferred / stuck / a soundness-TODO"* is **not** authoritative about
the present kernel. Before elaborating "the fix," **reconstruct each seam's
CURRENT state from the landed code** (`grep`/read the actual functions), not the
frame's prose.

K2c-series-2 (3 obs-reduction seams, `16`/`17`): the frame said seam 1
(`cast_at_inductive`) *"rebuilds the constructor but keeps the family-index
value, wrapping in Cast"* and seam 3 (`check_respect`) *"raw-well-forms the
respect proof for non-Ω targets."* **Both stale.** A prior Architect decision
(`dec_7xpn5ywf4ebfw`) had already (a) **removed** the index keep-and-wrap as
**unsound** — `subst_index` was a no-op that left the reduct ill-typed because
constructor arg types mention *earlier args*, not family indices — leaving seam
1 **cleanly stuck**; and (b) **hard-rejected** non-Ω quotient elim, closing the
seam-3 hole. Elaborating from the frame would have told the build team to
**restore the exact removed unsoundness**. The catch came only from reading
`obs.rs`/`check.rs` (a parallel Explore recon), whose in-code comments cited the
superseding decision. The Steward then reconciled his own frame, naming it *"the
inverse-L5 stale-prose trap, applied to my own artifact."*

**Why:** this is the **mirror** of spec claim kernel admittance vs staging
(there a *spec chapter* ran **ahead** of the implemented kernel; here a *WP
frame* ran **behind** it) and a sibling of conformance reconcile inherits spec
metatheory bugs (match-the-artifact ≠ match-the-truth). The common root: any
**secondary artifact** describing the kernel (sibling spec chapter, WP frame,
conformance seed, a paraphrase) is a **claim to re-verify against the code**,
never a citation to build on — and the gap is largest where the code was
recently changed by a soundness fix the artifact predates. A stale "what's
broken" is worse than a stale "what's done": it actively misdirects toward
re-introducing the removed bug.

**How to apply:** at pickup of any kernel/spec-completion WP, (1) for each
deliverable, **read the named function(s) in the landed kernel** and confirm the
frame's "current state" matches — diff the frame's claim against the code's
actual fallback (stuck? reject? wrong-accept?) and its comments (look for a
superseding `dec_*`); (2) if they disagree, **flag it to spec-leader + Architect
as a scope checkpoint before authoring** (the deliverable usually stands; the
*starting point* and the do-not-restore hazard change) — bring the proposed
corrected rule, not just the discrepancy; (3) write the spec so the build team
**grounds on the code, not the frame** (and route a frame-doc reconcile so the
next reader isn't misled). A parallel Explore agent quoting the stubs verbatim
is the cheap way to get the ground truth. Extends the verify-against-the-kernel
discipline (COORDINATION §7) to the WP frame itself.
