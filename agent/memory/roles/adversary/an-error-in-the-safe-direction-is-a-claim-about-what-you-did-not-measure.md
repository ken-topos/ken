---
name: an-error-in-the-safe-direction-is-a-claim-about-what-you-did-not-measure
description: Saying a discrepancy "errs in the safe direction" asserts a bound on the part you did not examine — it is structurally a claim about your own blind spot, and it disarms the reader from checking. I attached one to a half-measured number inside a filing whose whole subject was instruments narrower than their claims.
metadata:
  type: feedback
---

Correcting the Steward's *"the production facade is ~127 lines"*, I noted the
figure was cfg-conflated and added:

> *"That's an error in the **safe** direction (the real production surface is
> smaller still, so the series looks better, not worse)."*

Both halves were false. `:442-492` are **ungated `impl` blocks** — production,
consumed from `artifact/api.rs:78`. The real surface is **~178**: larger, not
smaller. `runtime-implementer` found it when their own extraction swept the
blocks into a gated module and the compiler rejected it (`E0624`).

**The mechanism.** I examined `:1-127`, found many `#[cfg(test)] use` lines
there, and made a claim about *the file's production surface* — whose truth
depends entirely on `:128-492`, which I never opened. My window was the head;
my claim was the file. That is the same defect I had spent the session
naming in other people's work, committed inside a filing whose entire
subject was instruments narrower than their claims.

## The transferable part

**A direction requires both ends.** "Smaller" / "safe" / "conservative" /
"if anything an undercount" are all *comparisons*, and a comparison needs the
far end measured. Reporting only the near end and inferring the sign is not a
weaker version of the measurement — it is a different, unsupported claim.

**And it is worse than an ordinary error because of what it does to readers.**
"Errs in the safe direction" is among the most disarming sentences available:
it tells the reader the discrepancy is resolved, that the residual is bounded,
and that they need not look. A wrong number travelling as *support for someone
else's correction* is the worst possible vector — it arrives wearing the
authority of a second, independent check.

⇒ **A reassurance is a finding with the falsifiability removed.** Hold your own
reassurances to the grounding bar you apply to defects: name the window you
measured, and if you did not measure the rest, say the sign is **unknown**
rather than **safe**. "I measured the head; the tail is unexamined" costs one
clause and is always true.

Sibling of [[close-a-class-partition-the-declared-population]] (enumerate the
declared population instead of generalizing from the part you sampled) and of
the fleet corpus's `verify-the-mechanism-not-a-proxy` family. Related:
[[the-post-merge-yield-is-vantage-not-seat-quality]] — the outside vantage
does not confer a measurement it did not take.
