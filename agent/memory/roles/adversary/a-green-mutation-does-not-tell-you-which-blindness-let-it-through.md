---
name: a-green-mutation-does-not-tell-you-which-blindness-let-it-through
description: "When you mutate to PROVE a blindness, green has many causes — the one you hypothesized, or the instrument never covering the target at all. The positive control is what discriminates, and without it the mutation reads as confirmation of whatever you already believed."
scope: roles/adversary
---

# A green mutation does not tell you which blindness let it through

Auditing `RT-SCALE-B`'s emission census, I hypothesised its per-file
`.define_function(` count was **region-blind** — production and `#[cfg(test)]`
occurrences summed into one number, so a compensating swap would hold it
constant. I built exactly that swap in `boundary_value_clif.rs`: move one
occurrence out of the test module, add one to the production region, total
unchanged. **Census green.** One step from filing.

The **positive control** killed it. Pushing the same file from 3 to 4
occurrences — an unbalanced change that *must* redden — was **also green**. The
file was not on the census roster at all. My "compensating swap survived" had
nothing to do with region-blindness; it was measuring a census that never looked
at that file.

The hypothesis turned out to be true anyway, but only on a **different** file:
adding the token inside a `#[cfg(test)]` function in `lowering/mod.rs` reddened
`0 → 1`. Right conclusion, and the evidence I first had for it was worthless.

## ⭐ Why this is not just "run a positive control"

[[a-negative-check-passes-for-any-reason-so-it-needs-a-positive-control]] covers
a check that passes. This is the adversary's own instrument failing the same
way, and it has a bias the generic version does not:

⛔ **The reasoning that selects the mutation target is the reasoning that
supplies the explanation.** I picked `boundary_value_clif.rs` *because* I
suspected it was poorly covered — so when the probe came back green, the cause I
already believed was sitting right there, pre-fitted. A green mutation is a
**disjunction** (my blindness ∨ some other blindness ∨ the instrument not
reaching here ∨ the probe never ran), and motivated reasoning collapses it to
the first disjunct for free.

⚠ This is the mirror of
[[a-mutation-that-reddens-does-not-confirm-which-detector-caught-it]]. Red needs
attribution; **green needs discrimination**. Neither is self-interpreting, and
the green case is worse because there is no error text to read.

## How to apply

1. **Every blindness probe ships with an unbalanced control on the same
   target** — a change that the instrument must reject. Run it *before* trusting
   any green. It is one extra run and it is the whole difference between a
   finding and a false one.
2. **If the control is also green, you have measured the wrong thing** — you now
   know the instrument does not see this target *at all*, which is usually a
   bigger fact than the one you set out to prove. Follow it.
3. **Re-site the experiment, do not re-argue it.** The hypothesis may still be
   correct; find a target the instrument demonstrably *does* cover and re-run
   there. A true claim with dead evidence is still unfiled.
4. **Report the near-miss.** It is cheap, it tells the reader which of your
   measurements are load-bearing, and it is the part that makes the surviving
   claim credible — same reason as
   [[an-error-in-the-safe-direction-is-a-claim-about-what-you-did-not-measure]].

Related: [[audit-a-detector-against-the-one-case-whose-answer-you-already-know]]
— the control here *is* the case whose answer you know, applied to your own
probe instead of someone else's checker.
