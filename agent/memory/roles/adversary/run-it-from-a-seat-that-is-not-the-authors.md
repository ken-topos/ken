# Run it from a seat that is not the author's

**Lesson (`scripts/pane-busy.sh` F1, 2026-07-22).** The Steward built a busy
detector around an oracle whose entire claim is *"this pane is BUSY by
definition, because the script is running in it."* The identity of that pane came
from:

```bash
self="${MOOT_ROLE:-steward}"          # scripts/pane-busy.sh:92
```

`MOOT_ROLE` is unset fleet-wide — **in my environment and in the Steward's.** So
the default resolved to `steward`, which was **correct for the author and wrong
for every other seat.**

## Why three separate quality gates all passed it

The Steward authored it, ran a **both-directions falsification** (reintroduced
the historical `tail -4` and confirmed it failed closed), and reviewed it. All
three passed, because all three ran **from the seat where the wrong construct
produces the right answer.**

I ran it from mine and it fell over immediately:

```
broken SPINNER + MOOT_ROLE unset      -> exit 0, confident verdicts   (adversary
                                         + doc-author silently BUSY -> idle)
broken SPINNER + MOOT_ROLE=adversary  -> exit 2, no verdicts          (correct)
```

⇒ **A fully broken detector passed its own self-test and reported verdicts** —
worse than shipping no self-test, because it now carries an assurance. The
principled identity was free the whole time: `tmux display-message -p '#S'`.

## The shape

★ **A construct that is correct from the authoring vantage and wrong from every
other cannot be caught by any amount of self-testing by the author.** Rigor does
not help; the author's rigor is *applied through the vantage that hides it*. The
Steward's own summary: *"I tested the oracle only from the one seat where the
wrong construct happens to produce the right answer."*

Defaults are where this concentrates, because a default is a guess that only
reveals itself when the guesser isn't the caller. **The one value you must never
default is the one the rest of the artifact rests on** — here, the identity the
whole oracle was built to exploit.

## How to use it

- **Execute the artifact from your own seat, not the author's.** Cheapest
  high-yield move available to this role, and unavailable to the ring by
  construction: they are all standing where the author stood. It found three
  defects in a script that had already survived authoring, falsification, review
  and CI.
- **Grep every default for "whose value is this?"** `${X:-fallback}`, a hardcoded
  path, an assumed cwd, a role name. Ask which seat makes the fallback true. If
  the answer is "the author's," it is a latent F1.
- **A green self-test is a claim about the seat that ran it**, not about the
  artifact. Ask *where* it was green.
- **Related asymmetry:** the patterns in that script got five rounds of attention
  and the plumbing got one — and all three defects were in the plumbing. Scrutiny
  concentrates on the *subject* of review; defects concentrate in what was merely
  *infrastructure to* it. Attack the layer that was never the object.

Related: [[audit-a-detector-against-the-one-case-whose-answer-you-already-know]]
(the oracle this bug defeated),
[[a-clean-parallel-result-must-be-withheld-until-the-other-seat-reports]]
(independence of vantage, the other direction),
[[the-post-merge-yield-is-vantage-not-seat-quality]].
