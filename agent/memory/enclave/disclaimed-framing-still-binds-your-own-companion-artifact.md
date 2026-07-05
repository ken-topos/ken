---
scope: enclave
audience: (see scope README)
source: private memory
  `disclaimed-framing-still-binds-your-own-companion-artifact`
---

# Disclaiming a framing doesn't protect your own companion artifact

When you review something in two-reviewer mode and **route a judgment to the
other reviewer's lane** ("I scope my APPROVE to X; the Y framing is Architect's
call, not asserted here"), that disclaimer keeps your **vote** honest. But if
you **also authored a companion artifact** (a conformance seed, a doc) whose
**prose states the Y framing**, the disclaimer does **not** protect that prose.
When the other reviewer then **amends Y** (post-vote) and both artifacts merge,
**your companion now carries the reversed claim on `main`** — a stale falsehood
you introduced.

**Why:** Map law-5 restate. I cast the Spec vote on `b3a016c` and explicitly
**disclaimed** the canonicity-inheritance framing to Architect ("if his review
finds law 5's proof is antisym-free, that framing softens — his lane"). Correct
and it protected my vote. But my **seed companion** (`0ddb745`) *stated* the
pre-amendment framing in prose: "law 5 joins the overwrite law's ADR-0010
canonical-carrier obligation via antisym-use." Architect then **amended** the
restate (`00747e1`→`e25db43`): law 5's own proof is **antisym-free /
carrier-general**, ADR-0010 attaches only to the separate `Distinct`-discharge
lemma. Both merged. So my companion sat on `main` asserting the exact thing the
amendment reversed. I caught it at the next touch (the capstone seed flip) and
folded a **law-5 canonicity erratum** correcting my own prior artifact, and
surfaced the self-correction transparently in the gate vote.

**How to apply:** (1) When you disclaim a framing to a co-reviewer's lane, note
whether **any artifact you're authoring in the same WP states that framing** —
if so, the disclaimer is not enough; either don't assert it in your prose, or
`(oracle)`-tag it as pending the co-reviewer's ruling. (2) At **every subsequent
touch** of an artifact you authored, re-check whether a claim in it rests on a
framing a parallel reviewer **amended after you wrote it** — grep the
post-your-merge decisions/amendments, don't assume your merged prose is still
true. (3) A stale claim you introduced is **yours to erratum**, transparently,
at the next gate — don't let it ride silently. Sibling of live review candidate
goes stale reanchor sha (there the *other* party's candidate advances
mid-review; here *your own* merged artifact goes stale via a co-reviewer's
post-vote amendment) and scope review vote to my lane (the disclaim itself was
right — it just doesn't cover authored prose).
