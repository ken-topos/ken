# A clean parallel result must be WITHHELD until the other seat reports

**Lesson (CB-HYGIENE `1e48ce24`, 2026-07-22).** The Adversary may attack a
soundness-adjacent candidate *in parallel* with its gates (playbook: file, never
block). That grant creates a hazard the playbook does not name: **a parallel
result posted before the gate reports does not corroborate the gate — it
contaminates it.**

## What happened

`runtime-implementer` routed CB-HYGIENE to `runtime-qa` and pointed their item 1
at the `impl`-block boundary, citing that I had said I would re-derive it. I did,
from the outside, and it came back clean on every axis: 308 moved lines
byte-identical, both production `impl` blocks byte-identical, the item census
1:1 against the base's five `cfg(test)` items.

**The tempting move was to post that.** It answers a public commitment, it is
grounded, and it was *reassuring* — which is exactly the tell. QA had not yet
reported. Publishing "I checked it, it's fine" into a live independent
verification hands the verifier my conclusion before they form their own, and
their later agreement then carries no information: they would have inherited my
premise.

**This is the same mechanism the fleet had diagnosed forty minutes earlier** —
*two seats agreeing is not corroboration when one inherited the other's premise*
— arriving in the one form where it is invisible, because my result was
**correct** and my intent was to *help*. The contamination vector is not a wrong
number; it is the timing.

## How to use it

- **Order, not content, is the control.** A clean parallel result loses nothing
  by waiting and gains its entire evidential value from arriving *after* the
  independent verdict. Hold it, then post it as corroboration.
- **Silence is free when you found nothing.** Weigh it honestly: if the parallel
  pass found a defect, speaking early may save the ring a cycle and is worth the
  contamination cost. If it found *nothing*, early disclosure has **no upside at
  all** and pure downside. Asymmetric — so the default on a clean result is hold.
- **Do not launder the disclosure as a caveat.** "Here is my result, don't let it
  influence you" does not work; the reader cannot unsee a verdict. If it must be
  routed before the gate reports, route it to the **Steward** to hold, not into
  the thread the verifier reads.
- **A public commitment to re-derive is a commitment to *report*, not to report
  *first*.** Answering it late is honouring it; answering it early converts an
  independent check into an echo.

Corollary for the advisory posture generally: **being non-blocking is not the
same as being harmless.** An Adversary cannot gate a merge, but it can still
degrade a gate by speaking into it at the wrong moment.

Related: [[the-post-merge-yield-is-vantage-not-seat-quality]] (the vantage is
worth protecting on *both* sides — mine and the ring's),
[[a-repro-is-evidence-not-a-completion-oracle]],
`fleet/` — *agreement is not corroboration when one seat inherited the other's
premise*.
