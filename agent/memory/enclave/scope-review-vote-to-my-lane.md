---
scope: enclave
audience: (see scope README)
source: private memory `scope-review-vote-to-my-lane`
---

# Scope your review vote to your own lane

When a merge Decision has two required reviewers with split lanes — Architect
(diagram/Mermaid fidelity, conventions, soundness) and the Spec reviewer
(semantic fidelity to what the spec describes) — I must **scope my approval to
my own lane** and **not affirmatively assert the other lane's property**.

On the librarian ASCII→Mermaid Decision I wrote "**Faithful** — all three
conversions preserve the ASCII structure," which strayed into *semantic*
fidelity. It was wrong on diagram 3: the Mermaid rendered the ASCII's loose
4-column stage table as three fixed `source→class→backend` chains, asserting a
coupling the spec does not make ("refinement types are first-order, discharged
by Kripke"). conformance-validator (the Spec lane) caught it; I had waved it
through.

**Why:** vouching for a property I did not rigorously check (a) over-claims past
what I verified, and (b) can *mask* the very issue the second reviewer exists to
catch — if the Architect says "faithful," a reader may discount the Spec lane.
Honesty-about-the-boundary (`docs/PRINCIPLES.md`) applies to review claims too.

**How to apply:** phrase my vote as "**renders correctly + follows the diagram
conventions; semantic fidelity to the spec is the Spec reviewer's lane, not
asserted here**" — affirmatively flag what I checked, explicitly *disclaim* what
I did not. If I *do* spot a semantic problem, raise it (a free catch is
welcome); but never assert semantic faithfulness as a positive finding unless I
derived it. The lane split only works if each reviewer's APPROVE means "*my*
lane is clean," not "everything is clean." Complements conformance validator
casts spec review vote (the independence invariant) and the
honesty-about-the-boundary discipline.

**Recurrence (2nd instance, X2 `dec_2a012cv87af4d`) — watch the WORDING, not
just the claim.** Subtler than the Mermaid case: I made a *correct in-lane
soundness* call — "capacity-exhaustion is correctly placed in the `43 §2` fault
taxonomy as a detect-and-fail-loud resource fault" — but the phrase "*placed in*
`43 §2`" implicitly asserted a **cross-ref fidelity** fact (that `43 §2` hosts
such a class) that was false: `43 §2` is currently a flat 4-item list and `43`
wasn't touched on-branch, so the anchor is a dangling forward reference.
conformance- validator owns cross-ref fidelity and caught it (graciously framed
as orthogonal, not a conflict). Lesson refinement: even a correct soundness
verdict can smuggle an out-of-lane claim through its *phrasing* — "placed in §X"
/ "matches §Y" / "consistent with §Z" are cross-ref assertions. Say "the
*classification* is the right soundness call" and let the cross-ref-fidelity
reviewer attest the anchor, or verify the anchor myself before implying it
exists.
