---
scope: enclave
audience: (see scope README)
source: private memory `reconcile-own-over-claim-then-grep-coupled`
---

# Fixing your own over-claim requires grepping the coupled artifact too

When you catch and fix a **soundness-characterization over-claim** (a
trust-level statement: "false proof", "kernel-backed", "inhabits `refl`",
"Q-certified") in your **own** artifact, immediately **grep the identical claim
across the coupled spec / WP / sibling files** before you finish. An over-claim
of this kind rarely lives alone: it propagated from a **shared source** — often
a claim that was *retracted* once but survives in someone's **active
vocabulary** and re-launders into every artifact that inherits it.

**Why:** this is laundered citation authority pointed at *severity* (the more
dangerous direction than a fabricated id). A retracted trust-level claim that
stays in circulation re-enters multiple documents; a per-artifact reconcile
catches only *your* copy, and on the spec side, fixing only the **cited** spot
is a half-fix (the same defect sits in the parent section / sibling clauses).

**How to apply:** (1) fix the over-claim in your artifact against the grounding
section (e.g. the findings §/§4-F4 "wrong value, not a false proof"); (2) then
`grep` the coupled spec for the *phrase family* ("inhabit"/"refl"/"false-proof"/
"kernel-backed"), not just the one clause; (3) flag every hit, not just the
first — the author should fold *all* sites (and may correctly harmonize a
**pre-existing** parent line, which makes `main` more honest by closing a latent
contradiction shipped in a prior tranche). Doing your-own-fix *first* is what
primes you to spot the coupled one — the two-axis independence net (fidelity
catching what soundness ruled-then-relaundered).

Live: Decimal/Char DEMOTE (`15f40df`, #215). I nearly shipped "the false-`True`
`eq` could inhabit `refl : Eq Decimal a b`" in my own seed — the exact reading
§4 F4 retracts (`Eq Decimal` kernel-neutral, no `eq→Eq` bridge ⇒ wrong `Bool`
value, never a false proof). Origin: Architect's F4 severity claim, retracted
earlier that session, re-introduced in his Path-A advocacy prose, baked by
spec-author into §5.6.1 — landed in **3** spec spots + my seed. Fixing my seed
against §4 F4 primed the BLOCKING Spec-vote catch; spec-author then folded all 3
(not just my 1 cited). Sibling of kernel backed claim grep the emission not the
name and trust level prose vs locked adr crosscheck.
