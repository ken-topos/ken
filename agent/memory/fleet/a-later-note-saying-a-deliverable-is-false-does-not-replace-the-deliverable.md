---
scope: fleet
audience: (see scope README) — anyone folding a review correction into a
  frame, WP, or spec section; anyone reading an AC that has a clarification
  appended below it
source: RT-FNSPLIT-B2V Architect blocks, 2026-07-25; confirmed again
  RT-FNSPLIT-B2F, 2026-07-28
---

# A later note saying a deliverable is false does not replace the deliverable

When a reviewer corrects a frame, **edit the operative deliverable in
place**. Appending a clarification that says the earlier text is false does
**not** replace it: both readings now live in the document, and the
**superseded one is the one positioned to be obeyed**, because construction
authority (the deliverable's own table, the AC set) is what an implementer
reads *first* and a clarification hundreds of lines below is read *second,
if at all*.

**Why:** two of the Architect's three blocks on the `RT-FNSPLIT-B2V` recut
(`docs/program/wp/RT-FNSPLIT-B2V-executable-value-abi.md`, 2026-07-25) had
this single cause. `D4`'s table still required four dispositions and
defined *represented immediate* as "payload fits the tagged word directly"
— a definition the recut's own clarification block declared false for
`RepresentedImmediate { spill: Some(..) }`. Separately, RETAIN still froze
the "64/112 layout change" that the recut's promoted wide-`Int` obligation
necessarily changes. Architect, exactly: *"A later note saying the earlier
deliverable is false does not replace the deliverable."* Appending **feels
like faithful transcription of the reviewer's words** and is in fact leaving
the defect operative — that is why it repeats.

## ⭐⭐ Confirmed again 2026-07-28 — self-authored, correct, and old

`RT-FNSPLIT-B2F`'s correction block at the very TOP of the frame recorded
that `lower_expr` has **61** call sites, not 59. Hours later the
implementer read `AC-5`'s own heading — *"all **59** calls"* — and reported
the frame as defective. ⛔ **Both numbers were mine; the banner was right;
the reader still got the stale number.**

⇒ ⭐ **Position does not save a correction.** Even a banner *above
everything* loses to the operative line, because **a reader who goes to an
AC to learn what the AC requires reads the AC** — not the document's
preamble. The banner is a supplement, never a substitute, and *"I already
corrected that at the top"* is not a defence.

⭐ **The fix that generalizes: delete the count from the requirement.** The
AC now says *every* call **at whatever count the tokenized derivation
returns on your own base**, and names the derivation as the only pin. ⇒ A
number in a requirement rots on the next merge and each fix mints a fresh
one to rot; **an obligation phrased over a derivation cannot go stale.**

**How to apply:** on every fold, (1) edit the requirement's own text —
replace the table, the AC sentence, the RETAIN bullet; (2) demote the
clarification to *explaining* that text rather than contradicting it; (3)
run a **whole-frame reconcile as part of the fold, not a step after it** —
re-read every deliverable, AC, and RETAIN list and confirm none still
states the superseded contract, then grep the old wording to prove it; (4)
leave the old phrasing only where it is explicitly marked superseded.
Sibling of
[[amending-a-frame-mid-flight-must-sweep-its-guardrails-section]] — same
shape: the header changed, the body did not. The pattern recurs wherever a
frame's status, retain list, or pinned number is edited without sweeping the
prose that restates it.
